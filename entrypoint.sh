#!/bin/sh

# Create the Nginx webroot directory if it doesn't exist
mkdir -p /var/www/html

# Redirect Nginx logs to Docker stdout/stderr
ln -sf /dev/stdout /var/log/nginx/access.log
ln -sf /dev/stderr /var/log/nginx/error.log

# Add the cron job, redirecting its output to container's stdout/stderr
# Note: Using /proc/1/fd/1 and /proc/1/fd/2 assumes entrypoint.sh is PID 1.
echo "0 */8 * * * /usr/local/bin/run_wiki_itn.sh >/proc/1/fd/1 2>/proc/1/fd/2" | crontab -

# Display current crontab entries for verification during startup
echo "Current crontab entries:"
crontab -l

# Start cron daemon in the background
echo "Starting cron daemon in background..."
cron

# Start Nginx in the foreground
echo "Starting Nginx in foreground..."
nginx -g 'daemon off;'
