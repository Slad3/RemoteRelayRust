cargo build --target=x86_64-unknown-linux-gnu --release
sudo cp scripts/RemoteRelayRust.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable RemoteRelayRust.service
sudo systemctl start RemoteRelayRust.service
sudo systemctl status RemoteRelayRust.service
