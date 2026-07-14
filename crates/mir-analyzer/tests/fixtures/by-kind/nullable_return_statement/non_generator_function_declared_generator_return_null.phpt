===description===
A function with no `yield` in its own body (not a generator) declared to
return `Generator` must actually return a `Generator` object — `return null;`
is still invalid, unlike the generator case in
`generator_return_type/native_hint_return_null_is_valid.phpt`.
===file===
<?php
function example() : Generator {
    return null;
}
===expect===
InvalidReturnType@3:4-3:16: Return type 'null' is not compatible with declared 'Generator'
