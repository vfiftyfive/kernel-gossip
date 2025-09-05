# "Your infrastructure is talking behind your back!" - Talk Outline

## 📋 Talk Structure (30 minutes)

### 🎪 **Act I: The Lie** (5 minutes)
- **Hook**: "How many of you trust your metrics?" 
- **The Problem**: Metrics show healthy, reality is different
- **Demo Setup**: Show standard Kubernetes observability
- **Transition**: "But what if I told you your kernel knows the truth?"

### 🔍 **Act II: The Hidden Cascade** (10 minutes)
- **The Journey Down**: From Pod to Syscalls
- **Demo 1**: Pod Birth Certificate - "847 syscalls to start nginx!"
- **Education**: Demystify the Kubernetes → Kernel translation
- **Wow Moment**: Make the invisible visible

### 👂 **Act III: The Kernel Whispers** (10 minutes)
- **The Journey Up**: From Kernel to Kubernetes
- **Demo 2**: CPU Throttle Detection - "Metrics lie, kernel knows!"
- **eBPF Introduction**: "Superpowers for observing the kernel"
- **Live Demo**: Real-time kernel signals → Kubernetes insights

### ⚡ **Act IV: The eBPF Magic** (3 minutes)
- **How It Works**: eBPF + Pixie simplified
- **The Architecture**: Webhook → Operator → Insights
- **Code Glimpse**: PxL script magic

### 🚀 **Act V: Your Turn** (2 minutes)
- **Call to Action**: You can build this!
- **Key Takeaways**: The three truths
- **Resources**: Where to start

---

## 🎯 Learning Objectives
By the end of this talk, attendees will:
1. **Understand** the gap between Kubernetes metrics and kernel reality
2. **Appreciate** eBPF's power for kernel observability
3. **See** practical application of kernel-aware infrastructure
4. **Know** how to start building similar solutions

## 💫 Wow Moments Planned
1. **"847 syscalls just to start nginx!"**
2. **"Metrics show 48% CPU but kernel shows 92.3% throttling!"**
3. **Real-time kernel event → Kubernetes insight in <2 seconds**
4. **Live PxL script execution showing eBPF in action**

## 🎪 Audience Engagement Strategy
- **Questions**: Interactive polls and rhetorical questions
- **Analogies**: Complex concepts explained with simple metaphors
- **Visuals**: Every technical concept gets a diagram or demo
- **Progression**: From familiar (K8s) to novel (eBPF) to practical (solutions)