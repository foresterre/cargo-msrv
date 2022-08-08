# Output format: human

This is the default output format. It can also be specified using the `--output-format human` option.

The output of the 'human' output format is intended to be interpreted by humans. It uses colour and custom printed
layouts to convey its information to the user. 

# Output by subcommand

## \# cargo msrv (find)

The output shows for each checked toolchain whether it is determined to be compatible or not.
Upon completion, a summary is shown which contains information on the search space considered, the search method used,
the compiler target and of course the MSRV.

### Example 1

In this example, we'll try to find the MSRV for an example project.

TODO screencast

From the above summary, we can determine that the MSRV is `1.42`.

### Example 2

This example showcases a case where there's no valid MSRV.  

TODO screencast

From the error message we learn that this is because of a nightly compiler feature, which is not available on stable.

## \# cargo msrv list

TODO

## \# cargo msrv set

TODO

### Example 1

TODO example

## \# cargo msrv show

TODO

### Example 1

TODO example

### Example 2

TODO example

## \# cargo msrv verify

TODO

### Example 1

TODO example

### Example 2

TODO example
