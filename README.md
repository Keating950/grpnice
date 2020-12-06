# grpnice

```text
grpnice: Adjusts niceness for the given PID's autogroup.

USAGE:	grpnice [-n (adjustment)] (PID)
n:	Added to the process group's niceness. Must be an integer. Defaults to 10.
PID:	PID to adjust.
-h:	Print this help message.
-v:	Print version info.
```

## Rationale
Many Linux distributions enable process autogrouping by default. This means that the
scheduler evaluates priority based by process group rather than individual process
niceness. While this is generally good for enabling fair scheduling, it does mean that the
default `nice` command is mostly ineffective in these cases. `grpnice` addresses this
issue by working at the autogroup level. For more info and discussion, see [this Reddit thread](
https://www.reddit.com/r/linux/comments/d7hx2c/why_nice_levels_are_a_placebo_and_have_been_for_a/).
