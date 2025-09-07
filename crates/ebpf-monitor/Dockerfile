# Custom eBPF monitor image with bpftrace and curl
FROM quay.io/iovisor/bpftrace:latest

# Install curl for webhook capability (Ubuntu-based image)
RUN apt-get update && \
    apt-get install -y curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Verify installations
RUN bpftrace --version && curl --version

ENTRYPOINT ["/bin/sh"]