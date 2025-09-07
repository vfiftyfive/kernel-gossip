use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use anyhow::Result;
use tracing::{info, error};

pub struct BpftraceProcess {
    child: Child,
    lines: Lines<BufReader<tokio::process::ChildStdout>>,
    stderr_lines: Lines<BufReader<tokio::process::ChildStderr>>,
}

impl BpftraceProcess {
    pub async fn spawn(script: &str) -> Result<Self> {
        info!("🔧 Spawning bpftrace with script length: {} bytes", script.len());
        
        let mut child = Command::new("bpftrace")
            .args(["-e", script])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);
        let lines = reader.lines();
        let stderr_lines = stderr_reader.lines();

        Ok(BpftraceProcess { 
            child, 
            lines,
            stderr_lines
        })
    }

    pub async fn next_line(&mut self) -> Result<Option<String>> {
        Ok(self.lines.next_line().await?)
    }

    pub async fn check_stderr(&mut self) -> Result<()> {
        // Non-blocking check for stderr output
        if let Ok(Ok(Some(line))) = tokio::time::timeout(
            tokio::time::Duration::from_millis(10), 
            self.stderr_lines.next_line()
        ).await {
            error!("🚨 bpftrace error: {}", line);
        }
        Ok(())
    }

    pub async fn wait(&mut self) -> Result<()> {
        let status = self.child.wait().await?;
        info!("🏁 bpftrace process exited with status: {:?}", status);
        
        // Read any remaining stderr messages
        while let Ok(Some(error_line)) = self.stderr_lines.next_line().await {
            if !error_line.is_empty() {
                error!("🚨 bpftrace final error: {}", error_line);
            }
        }
        
        Ok(())
    }
}