Vagrant.configure("2") do |config|

	# Specify the base box
	config.vm.box = "ubuntu/trusty64"

  config.vm.synced_folder "apps/", "/home/vagrant/apps"

  # VM specific configs
  config.vm.provider "virtualbox" do |v|
  	v.name = "Arduino Rust"
  	v.customize ["modifyvm", :id, "--memory", "1024"]

    # Required to enable USB
    v.customize ["modifyvm", :id, "--usb", "on"]
    v.customize ["modifyvm", :id, "--usbehci", "on"]
  end

  # Shell provisioning
  config.vm.provision "shell" do |s|
  	s.path = "provision/provision.sh"
  end
end
