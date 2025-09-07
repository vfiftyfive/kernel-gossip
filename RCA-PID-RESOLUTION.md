# Root Cause Analysis: PID Resolution Failure

## Executive Summary
The kernel-observer successfully detects container lifecycle events via eBPF but fails to resolve host PIDs to Kubernetes pod information, preventing PodBirthCertificate creation.

## Problem Statement
- **Symptom**: "Could not resolve PID 1685273 to pod information" errors
- **Impact**: No PodBirthCertificates created for new pods
- **Scope**: Affects all container birth detection functionality

## Root Cause Analysis

### 1. Namespace Mismatch
**Finding**: eBPF sees host namespace PIDs, Kubernetes API returns container namespace PIDs
- eBPF runs in host PID namespace (privileged DaemonSet)
- Container processes have different PIDs inside vs outside container
- Example: nginx PID 1 inside container, PID 1685273 on host

### 2. Cgroup Path Issues
**Current Implementation** (`pod_resolver.rs:51-112`):
```rust
// Attempts to read /proc/{pid}/cgroup
let cgroup_path = format!("/proc/{pid}/cgroup");
let cgroup_content = tokio::fs::read_to_string(&cgroup_path).await?;
```

**Problems**:
- Path exists but may not contain pod UID for short-lived processes
- GKE uses systemd cgroup driver with different path structure
- Container runtime processes (runc) exit before cgroup stabilizes

### 3. Timing Issues
**Observation**: Runtime processes are ephemeral
- runc process lifetime: ~123ms
- Cgroup creation happens asynchronously
- Pod UID not immediately available in /proc/PID/cgroup

### 4. Command Name Truncation
**eBPF Issue**: `comm` field shows "6" instead of "nginx"
- bpftrace comm field limited to 16 chars
- Truncation happens at kernel level
- Makes process identification unreliable

## Technical Deep Dive

### Current Flow (FAILING)
```
1. eBPF detects: CONTAINER_MAIN pid=1685273 comm=6
2. Parser spawns async task with 50ms delay
3. Attempts to read /proc/1685273/cgroup
4. Fails to find pod UID (process may have exited or cgroup not ready)
5. Kubernetes API query fails
6. No PodBirthCertificate created
```

### Why It Fails in GKE
- GKE uses containerd with systemd cgroup driver
- Cgroup paths: `/sys/fs/cgroup/systemd/kubepods.slice/kubepods-burstable.slice/`
- Pod UID embedded differently than expected
- Host mount of /proc may not reflect container cgroups correctly

## Idiomatic Kubernetes Solution

### Design Principles
1. **Event-Driven**: Use Kubernetes informers instead of /proc parsing
2. **Declarative**: Match processes to pods via labels/annotations
3. **Resilient**: Handle namespace boundaries properly
4. **Observable**: Emit metrics and events

### Proposed Architecture

```
┌─────────────────────────────────────────────────────┐
│                  kernel-observer                     │
│                   (DaemonSet)                        │
├─────────────────────────────────────────────────────┤
│                                                      │
│  1. eBPF Component (bpftrace)                       │
│     - Captures: PID, PPID, cgroup_path, container_id│
│     - Emits: Structured events with metadata        │
│                                                      │
│  2. Container Runtime Client                        │
│     - Connects to: /run/containerd/containerd.sock  │
│     - Queries: Container ID → Pod metadata          │
│     - Caches: PID → Container → Pod mappings        │
│                                                      │
│  3. Kubernetes Informer                             │
│     - Watches: Pod create/update/delete events      │
│     - Maintains: Local pod cache                    │
│     - Provides: Pod UID → Pod object mapping       │
│                                                      │
│  4. Correlation Engine                              │
│     - Matches: eBPF events to pod cache             │
│     - Enriches: Events with K8s metadata            │
│     - Sends: Webhooks to operator                   │
│                                                      │
└─────────────────────────────────────────────────────┘
```

### Implementation Strategy

#### Phase 1: Enhanced eBPF Data Collection
```c
// Capture container ID from cgroup path
tracepoint:cgroup:cgroup_attach_task {
    $cgroup_path = str(args->path);
    if ($cgroup_path ~ "docker-" || $cgroup_path ~ "containerd-") {
        // Extract container ID (first 12 chars after prefix)
        printf("CONTAINER_ATTACH pid=%d container_id=%s path=%s\n", 
               args->pid, $cgroup_path, $cgroup_path);
    }
}
```

#### Phase 2: Container Runtime Integration
```rust
use containerd_client::Client;

pub struct ContainerResolver {
    client: Client,
    cache: Arc<RwLock<HashMap<String, PodInfo>>>,
}

impl ContainerResolver {
    pub async fn resolve_container_to_pod(&self, container_id: &str) -> Option<PodInfo> {
        // Query containerd for container labels
        let container = self.client.get_container(container_id).await?;
        let labels = container.labels();
        
        // Extract Kubernetes metadata
        let pod_name = labels.get("io.kubernetes.pod.name")?;
        let pod_namespace = labels.get("io.kubernetes.pod.namespace")?;
        let pod_uid = labels.get("io.kubernetes.pod.uid")?;
        
        Some(PodInfo {
            name: pod_name.clone(),
            namespace: pod_namespace.clone(),
            uid: pod_uid.clone(),
        })
    }
}
```

#### Phase 3: Kubernetes Informer
```rust
use kube::runtime::{watcher, WatchStreamExt};

pub struct PodWatcher {
    pods: Arc<RwLock<HashMap<String, Pod>>>,
}

impl PodWatcher {
    pub async fn start(&self, client: Client) {
        let api: Api<Pod> = Api::all(client);
        let stream = watcher(api, Default::default()).applied_objects();
        
        while let Some(pod) = stream.try_next().await? {
            let mut cache = self.pods.write().await;
            if let Some(uid) = &pod.metadata.uid {
                cache.insert(uid.clone(), pod);
            }
        }
    }
}
```

#### Phase 4: Correlation Logic
```rust
pub async fn correlate_event(&self, ebpf_event: EbpfEvent) -> Option<EnrichedEvent> {
    match ebpf_event {
        EbpfEvent::ContainerAttach { pid, container_id, .. } => {
            // Get pod info from container runtime
            let pod_info = self.container_resolver
                .resolve_container_to_pod(&container_id)
                .await?;
            
            // Get full pod object from cache
            let pod = self.pod_cache.get(&pod_info.uid).await?;
            
            // Create enriched event
            Some(EnrichedEvent {
                pid,
                pod_name: pod_info.name,
                pod_namespace: pod_info.namespace,
                pod_uid: pod_info.uid,
                container_name: extract_container_name(&pod, &container_id),
                timestamp: Utc::now(),
            })
        }
        _ => None,
    }
}
```

### Alternative: CRI API Approach
```yaml
# Use CRI (Container Runtime Interface) for standard K8s integration
apiVersion: v1
kind: ConfigMap
metadata:
  name: kernel-observer-config
data:
  runtime_endpoint: "unix:///run/containerd/containerd.sock"
  runtime_type: "containerd"  # or "cri-o", "docker"
```

### Benefits of This Design
1. **Reliable**: Direct container runtime integration
2. **Fast**: Local caching reduces API calls
3. **Accurate**: No PID namespace confusion
4. **Portable**: Works across different container runtimes
5. **Idiomatic**: Uses standard Kubernetes patterns

### Immediate Tactical Fix
For quick resolution, implement container ID extraction from cgroup events:

```rust
// In parser.rs
if line.contains("CONTAINER_ATTACH") {
    let container_id = extract_field(line, "container_id=")?;
    
    // Use container ID to query K8s API with label selector
    let pods = Pod::list(&client, &ListParams {
        label_selector: Some(format!("io.kubernetes.container.name={}", container_id)),
        ..Default::default()
    }).await?;
    
    if let Some(pod) = pods.items.first() {
        // Create PodBirthCertificate
    }
}
```

## Recommendations

### Short Term (1-2 days)
1. Add container ID extraction to eBPF script
2. Implement containerd client for PID→Container resolution
3. Cache container→pod mappings locally

### Medium Term (1 week)
1. Implement full Kubernetes informer pattern
2. Add metrics for resolution success/failure rates
3. Create fallback resolution strategies

### Long Term (2 weeks)
1. Support multiple container runtimes (CRI abstraction)
2. Add distributed tracing for debugging
3. Implement pod lifecycle state machine

## Success Metrics
- PID resolution success rate > 95%
- PodBirthCertificate creation latency < 100ms
- Zero false negatives for pod creation events

## Conclusion
The current approach of reading /proc/PID/cgroup is fundamentally flawed due to namespace boundaries and timing issues. The idiomatic solution is to integrate directly with the container runtime and use Kubernetes informers for reliable pod resolution.