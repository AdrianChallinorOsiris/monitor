# Monitor
## A discreet remote data provider for CONKY

Monitor is a small executable program that provides a number of endpoints that a [Conky server]
 (https://en.wikipedia.org/wiki/Conky_(software)) can call to gather realtime monitoring data for display.

 The enclosed screen shot shows 7 different boxes displayed on one background screen:
 * On the RHS, BASTET is the name of the host machine
 * In the centre, the four Odroid units are single board ARM computers or SBCs
 * On the LHS, ISIS is a desktop PC.

## Running monitor
Monitor runs as an executable program. It will run quite happily with no parameters at all. However, it can be customised by command line flags:
```monitor 0.1.0

AUTHOR: Adrian Challinor
Remote monitor server, mainly for CONKY
 
USAGE:
    monitor [OPTIONS]  

FLAGS:  
    -h, --help       Prints help information  
    -V, --version    Prints version information  

OPTIONS:    
    -a, --address <address>    The IP address to bind to [default: 0.0.0.0]
    -p, --port <port>          The IP port to bind to [default: 8000]
    -w, --workers <workers>    The number of concurrent worker threads [default: 5]
```

To use a remote monitor, firstly, the monitor application must be installed and running on the remote computer. Sounds obvious, but the second part, it must be running, is a prequisite. It is left for the user to decide how to run it. You can start it on your desktop as you login; you can run it interactively (when you will see all the calls being reported). 

## Interface to CONKY
Once running, a Conky display can be created using **exec** and **execi** verbs. The *curl* command is used to connect to the remote monitor. The endpoint uri determines what dat awill be extracted and returned. For example:
    SERVER: ${execi 3600 curl http://mybox:8000/name}`

will connect to the monitor running on the standard port, 8000, on the box called **mybox**. You can use fully qualified names, short names from your /etc/hosts file, or even IP addresses. The *name* end point specifies that you want to see the computer name.

For my Odroid SBCs, the full script for Conky is:
```
${color orange}SERVER:${color3}${exec curl http://o0:8000/name}
${color orange}Version : ${color2}${exec curl http://o0:8000/os/versionname}
${color orange}OS      : ${color2}${exec curl http://o0:8000/os/codename}
${color orange}Kernel  : ${color2}${exec curl http://o0:8000/uname/r }
${color orange}Uptime  : ${color2}${execi 20 curl http://o0:8000/uptime }
${color orange}CPU Load Av.: ${color2}${execi 60 curl http://o0:8000/loadavg}
${color orange}CPU load: ${color2}${execi 60 curl http://o0:8000/cpuload}
${color orange}Core 0: ${color2}${execi 60 curl http://o0:8000/temp/0}°C ${
```


I know, you are going to get long lines with Conky. Sorry about that, but frankly, tough.

## What can be monitored
Monitor is a work in progress. I created it to give me the information I wanted to see, but it is being upgraded and enhanced all the time. All returns are formatted as sent as character strings. The curent list of end points are:

```
ENDPOINT                Description
==============          =====================================
/status                 Returns the fact Monitor is running and its version

/boot                   Boot time

/os/name                The name of the current os, eg Linux

/os/version             The version as numeric values, eg 18.10

/os/versionname         The version as a name, eg "18.10 Cosmic Cuttlefish"

/os/codename            The code name, eg Cosmic

/temp/<ID>              Applies to ARM computers only.
                        See Sensors below

/sensors/<chip>/<param> Applies AMD motherboards confirmed.
                        See sensors section below

/uptime                 Formatted string of the uptime of the box

/aptcheck               Applies to Ubuntu boxes only
                        Reports if any updates are pending

/aptcheckbrief          Like aptcheck, it reports the changes pending, but as a 
                        single line. It does split the changes to show how many are 
                        security release.

/reboot                 This does NOT reboot the remote server! 
                        It reports if the automatic installation of required updates 
                        means that the server has to be rebooted. 

/disk/<disk>/<param>    The <disk> is the name of the disk you want info about
                        but omit the leading "/". For the system disk, use the special name "root".

                        <param> is parameter you want info about:
                            avail       Available disk space
                            total       Total disk space
                            free        Amount that is free
                            freep       The percentage that is free
                            usedp       The percentage that is used
                            files       The number of files (inodes)

/cpu                    Total CPU usage as a percentage. Note that getting
                        remote CPU usage by individual core is not (yet)
                        supported.

/cpuload                The load load as percentages of:
                        user, nice, system, interupt and idle process time

/loadavg                The average load over the last 1, 5 and 15 minutes

/memory                 The memory usage, as absolute numbers, of how much is 
                        used, and how much is free. 
            
/ip                     The main IP address of the server. For servers with 
                        multiple IP's, this will normally be the one which is 
                        used to connect to the internet. 

/port/<number>          Checks if there is a service on this port that is 
                        accepting connection. For example, 6379 will check for 
                        a Redis server, whereas 80 will check for a web server. 

/uname/<param>          Interrogate the uname command. The params relate to the 
                        infomation to return: 
                            n   - Node name 
                            s   - System name - normally Linux
                            r   - OS Version
                            v   - The full OS Version Name 
                            m   - Machine architecture
```

## Hardware Sensors
Not all PC's are created equally. Indeed, not all Linux based PC's are the same. Leastways, not when it comes to handling the hardware sensor values. For example, all though some of my SBC run Ubuntu, they do not have access to /usr/bin/sensors, nor to the sensorsd linkable library. Furthermore, diffrent motherboards and chipsets use different methods to report fan speed and cpu temperatures. Its pretty much a free for all out there.

Monitor approaches this in a pragmatic way. It provides two different interfaces:
    * /sensors is used for AMD motherboards. Intel - watch this space
    * /cpu  is used for ARM processors

###/sensors
The motherboard and different PCI boards can all provide different information and all have different ways of accessing the various parameters. They are accessed via an endpoint for each parameter. To find out what endpoints are available, we have provided a small executable that will list the availble
endpoints on your motherboard. **Don't copy mine, or anyone elses**. Use the
**sensors** program to see what is available. The output gives you the URI you need to call.


## Sensors 

To find out which sensors your system supports, you can ask **monitor** to probe them. To do this, run the 
program with the -s (--sensors) option: 

```
monitor -s 
```

On my system, **sensors** gives:

```
bastet : Listing all sensors in conky monitor format

http://bastet:8000/nouveau-pci-0100/GPU_core = +0.91
http://bastet:8000/nouveau-pci-0100/temp1 = +47.0°C
http://bastet:8000/k10temp-pci-00c3/CPU_Temp = +29.8°C
http://bastet:8000/it8721-isa-0290/12V = +11.65
http://bastet:8000/it8721-isa-0290/5V = +4.78
http://bastet:8000/it8721-isa-0290/Vcore = +1.36
http://bastet:8000/it8721-isa-0290/3_3V = +3.29
http://bastet:8000/it8721-isa-0290/VDDA = +2.52
http://bastet:8000/it8721-isa-0290/Vbat = +3.36
http://bastet:8000/it8721-isa-0290/CPU_Fan1 = 1163
http://bastet:8000/it8721-isa-0290/CPU_Fan2 = 890
http://bastet:8000/it8721-isa-0290/Chassis_Fan = 998
http://bastet:8000/it8721-isa-0290/CPU_Temp = +34.0°C
http://bastet:8000/it8721-isa-0290/M_B_Temp = +32.0°C
http://bastet:8000/nouveau-pci-0600/GPU_core = +0.90
http://bastet:8000/nouveau-pci-0600/temp1 = +50.0°C
http://bastet:8000/asus-isa-0000/cpu_fan = 0
http://bastet:8000/fam15h_power-pci-00c4/power1 = 86.88
```

### ARM processors
ARM systems, in fact all SBCs I have come across, do not support the sensors interface. Instead, the board data is acessed via the system files. Depending on your board and the operating system you are running, different configurations may be necessary.

For example, for the ODROID XU4, running Ubuntu 18.4 LTS the endpoints are:
```/temp/<ID>```

where <ID> is
    1.  Core 1
    2.  Core 2
    3.  Core 3
    4.  Core 4
    5.  GPU
Note that it has 8 cores, but they are split major and minor cores. The CPU die packages a majore and a minor core together as one, and the temperature of the two together is reported as a major core.

For Raspberry PI installation - *watch this space*

 ## Installation
 The **monitor** and **sensors** applications are RUST programs. 
 
 
 1. [Install RUST](https://www.rust-lang.org/tools/install). Don't panic. It is ridiculously simple to do.

 2. Clone the git repository (and Sensors) 
    git clone https://github.com/AdrianChallinorOsiris/monitor.git

 3. Change direcory to where the Monitor code is installed, you should see a src directory and some Cargo.toml files.
    cd monitor

 4. Build the application 
    cargo build --release

 5. If you get problems, contact me for assistance

 6. Otherwise, copy the file **./target/release/monitor** to wherever you want to run it from. You're now good to go. We suggest /usr/local/bin 

 7. To run this as a service a systemd definition file is provided  
    sudo cp monitor.service /etc/systemd/system/
    sudo systemctl start monitor 
    sudo systemctl enable monitor
    sudo systemctl status monitor 



To build a system that does not depend on GLIB
rustup target add x86_64-unknown-linux-musl --toolchain=nightly
cargo build --target x86_64-unknown-linux-musl --release
