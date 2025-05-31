#!/bin/sh
# Ensure the Nginx webroot directory exists
mkdir -p /var/www/html

# Fetch the Wikipedia page content and pipe it to the wiki-itn binary
# The output (Atom feed) is redirected to feed.xml in the Nginx webroot
curl -s https://en.wikipedia.org/wiki/Template:In_the_news | /usr/local/bin/wiki-itn > /var/www/html/feed.xml
