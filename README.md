# Licht
Utility for chaning laptop backlight brightness, supporting different stepping modes.


# Usage
```
licht 

USAGE:
    licht [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --device-name <DEVICE_NAME>    The backlight class device from sysfs to act on. E.g.
                                       intel_backlight If no device name is supplied and unless any
                                       other related flag is set licht will attempt to discover a
                                       backlight device in sysfs
    -h, --help                         Print help information

SUBCOMMANDS:
    get     
    help    Print this message or the help of the given subcommand(s)
    set     
```
```
licht-set 

USAGE:
    licht set [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --device-name <DEVICE_NAME>
            The backlight class device from sysfs to act on. E.g. intel_backlight If no device name
            is supplied and unless any other related flag is set licht will attempt to discover a
            backlight device in sysfs

        --min-brightness <MIN_BRIGHTNESS>
            Clamps the brightness to a minimum value [default: 0]

        --verbose
            Use verbose output

        --dry-run
            Do not write the new brightness value to the backlight device. dry-run implies verbose

        --all
            Operate on all backlight devices

    -h, --help
            Print help information

SUBCOMMANDS:
    absolute     Sets the current brightness value to <STEP>%
    blend        Maps the current birghtness value onto the function ratio*x^a + (1-m) *
                     (1-(1-x)^(1/b) and advances it <STEP>% on that function. Recommended parameters
                     for this function are ratio = 0.75, a = 1.8, b = 2.2. The argument for that
                     would be --blend (0.75,1.8,2.2) Enter the above function into e.g. desmos or
                     geogebra and change the parameters to your liking
    geometric    Multiplies the current brightness value by <STEP>%
    help         Print this message or the help of the given subcommand(s)
    linear       Adds <STEP>% to the current brightness value
    parabolic    Maps the current brightness value onto a the parabolic function x^exponent and
                     advances it <STEP>% on that function
```
```
licht-get-info 

USAGE:
    licht get info [OPTIONS] <--name|--class|--brightness|--percent|--max-brightness|--everything>

OPTIONS:
        --device-name <DEVICE_NAME>    The backlight class device from sysfs to act on. E.g.
                                       intel_backlight If no device name is supplied and unless any
                                       other related flag is set licht will attempt to discover a
                                       backlight device in sysfs
        --brightness                   
        --class                        
        --csv                          
        --everything                   
    -h, --help                         Print help information
        --max-brightness               
        --name                         
        --percent                      
```
```
licht-get-list 
List availble backlight devices

USAGE:
    licht get list [OPTIONS]

OPTIONS:
        --device-name <DEVICE_NAME>    The backlight class device from sysfs to act on. E.g.
                                       intel_backlight If no device name is supplied and unless any
                                       other related flag is set licht will attempt to discover a
                                       backlight device in sysfs
    -h, --help                         Print help information
```

# Example 
```
// Reduces brightness by 10%
licht set linear -10

// Reduces brigthness by 20% on the parabolic brightness curve x^exponent. 
// Actual brightness progression: 100% -> 64% -> 36% -> 16% -> 4%
licht set parabolic 2 -20

// Increases brightness of intel_backlight by 10% on the custom blend function 0.75*x^1.8 + (1-0.75)*(1-(1-x)^(1/2.2))
licht --device-name intel_backlight set blend 0.75 1.8 2.2 -10

// List all backlight deices
licht get list

// Get current brightness of default backlight
licht get info --brightness

// Get name, class, brightness, brightness percent and maximum brightness of intel_backlight
licht --device-name intel_backlight get info --everything
```
