# Licht
Utility for chaning laptop backlight brightness, supporting different stepping modes.


# Usage
```
licht 

USAGE:
    licht [OPTIONS] [STEP]

ARGS:
    <STEP>    The step used by the chosen stepping. By default it's +-% on the parabolic curve
              x^2 but could be a factor or a raw value. See the chosen stepping for details

OPTIONS:
        --device-name <DEVICE_NAME>
            The backlight class device from sysfs to control. E.g. intel_backlight

        --absolute
            Simply adds the raw <STEP> value onto the raw current brightness value

        --set
            Simply sets the current brightness value to <STEP>

        --geometric
            Multiplies the current brightness value by <STEP>%

        --parabolic <EXPONENT>
            Maps the current brightness value onto a the parabolic function x^exponent and advances
            it <STEP>% on that function

        --blend <(RATIO,A,B)>
            Maps the current birghtness value onto the function ratio*x^a + (1-m) * (1-(1-x)^(1/b)
            and advances it <STEP>% on that function. Recommended parameters for this function are
            ratio = 0.75, a = 1.8, b = 2.2. The argument for that would be --blend (0.75,1.8,2.2)
            Enter the above function into e.g. desmos or geogebra and change the parameters to your
            liking

        --min-brightness <MIN_BRIGHTNESS>
            Clamps the brightness to a minimum value [default: 0]

        --verbose
            Use verbose output

        --dry-run
            Do not write the new brightness value to the backlight device. dry-run implies verbose

        --list
            List availble backlight devices

    -h, --help
            Print help information
```

# Example 
```
// Reduces brigthness by 20% on the parabolic brightness curve. 
// Actual brightness progression: 100% -> 64% -> 36% -> 16% -> 4%
licht intel_backlight -20

// Custom brightness curve
licht intel_backlight -10 --blend (0.75,1.8,2.2)

// Geometric brightness curve with minimum min-brightness value
licht intel_backlight  10 --geometric --min-brightness 250
```
