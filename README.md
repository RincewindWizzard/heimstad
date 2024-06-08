# heimstad

Heimstad is (will be) a simple web UI for my smart home offering a
[rocket](https://rocket.rs/) powered website for a [MQTT](https://de.wikipedia.org/wiki/MQTT)
backend and some time series database for logging. (Might be PostgreSQL)

Features will include what I need and what I like.

See also the [Documentation](https://rincewindwizzard.github.io/heimstad/heimstad/index.html).

# Installation

Installation should be easy

apt update && apt upgrade -y && apt install -y curl wget gpg
wget -O- https://rincewindwizzard.github.io/heimstad/deb/KEY.gpg | gpg --dearmor > /etc/apt/trusted.gpg.d/heimstad.gpg
echo "deb [arch=amd64,signed-by=/etc/apt/trusted.gpg.d/heimstad.gpg]  https://rincewindwizzard.github.io/madriguera-web/deb/ ./" > /etc/apt/sources.list.d/heimstad.list
apt update
apt install heimstad