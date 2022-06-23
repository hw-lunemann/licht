# Licht
Utility for chaning laptop backlight brightness, supporting different stepping modes.


# Usage
```
licht 

USAGE:
    licht [OPTIONS] <DEVICE> <STEP>

ARGS:
    <DEVICE>    
    <STEP>      

OPTIONS:
        --dry-run                
        --exponent <EXPONENT>    [default: 2]
    -h, --help                   Print help information
        --stepping <STEPPING>    [default: parabolic] [possible values: absolute, geometric,
                                 parabolic]
        --verbose
```

# Example 
```
// Reduces brigthness by 20% on the parabolic brightness curve. 
// Actual brightness progression: 100% -> 64% -> 36% -> 16% -> 4%
licht intel_backlight -20
```
