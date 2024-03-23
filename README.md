# How and when to use
## When to use
This script provides a solution to the following problem. You have one public ip address, and multiple webservers on your LAN that you want to use with different domains. Especially, if you regularly want to deploy new domains with new webservers.

## How to use
This script only generates configuration files. For now, it generates nginx configuration files, and configuration files for [acme-redirect](https://github.com/kpcyrd/acme-redirect). 

To configure acme-redirect with nginx, please follow their guide.

When you run it the first time, it generates a home directory in /etc/web-distributor.

To configure it, edit /etc/web-distributor/config.toml. You can edit following variables:
- home: this defines where to put the webserver configuration files. defaults to /etc/web-distributor.
- acme_redirect_configs: where to put the acme-redirect configuration files. Defaults to /etc/acme-redirect.d.
- \[map\]: Here, you can put a list of your desired reverse proxies, separated by a linebreak. "example.com" = "127.0.0.1" would be a correct syntax.

You can also specify a different path for the configuration file like this: `web-distributor <config-file>`.

If you leave the defaults like this, you will have to manually include the configuration files to nginx. To do this, add the line `include /etc/web-distributor/nginx/*` to your nginx.conf.

Every time you run this script with a new configuration, run `acme-redirect renew` afterwards. Depending on how you configured acme-redirect, also run `systemctl reload nginx`.