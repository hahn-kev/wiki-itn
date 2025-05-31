# Stage 1: Builder for Rust application
FROM rust:latest AS builder

WORKDIR /usr/src/wiki-itn

# Install cron and curl
RUN apt-get update && apt-get install -y cron curl && rm -rf /var/lib/apt/lists/*

# Copy Cargo files and build dummy project to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy actual source code and build
COPY src ./src
RUN rm -f target/release/deps/wiki_itn* # Remove previous build artifacts
RUN cargo build --release

# Copy scripts
COPY run_wiki_itn.sh /usr/local/bin/run_wiki_itn.sh
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/run_wiki_itn.sh /usr/local/bin/entrypoint.sh

# Stage 2: Final image with Nginx
FROM nginx:latest

# Install cron and curl (needed for the scripts)
RUN apt-get update && apt-get install -y cron curl && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder stage
COPY --from=builder /usr/src/wiki-itn/target/release/wiki-itn /usr/local/bin/wiki-itn

# Copy scripts
COPY run_wiki_itn.sh /usr/local/bin/run_wiki_itn.sh
COPY entrypoint.sh /usr/local/bin/entrypoint.sh

# Nginx configuration will be copied in a later step.
# For now, create the directory where Nginx will serve files.
RUN mkdir -p /var/www/html

# Ensure scripts are executable (permissions might be lost during copy)
RUN chmod +x /usr/local/bin/run_wiki_itn.sh /usr/local/bin/entrypoint.sh

COPY nginx-site.conf /etc/nginx/conf.d/default.conf

HEALTHCHECK --interval=10s --timeout=5s --start-period=10s --retries=5 CMD curl -f http://localhost/feed.xml || curl -f http://localhost/ || exit 1

# Expose port 80 for Nginx
EXPOSE 80

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# CMD ["nginx", "-g", "daemon off;"] # This will be handled by entrypoint.sh
