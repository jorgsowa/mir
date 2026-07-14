===description===
A generator's `return <expr>;` is still checked, but against the declared
`TReturn` (4th) type param, not the whole `Generator` type.
===file===
<?php
/** @return Generator<int, string, mixed, bool> */
function gen(): \Generator {
    yield 'a';
    return "not a bool";
}
===expect===
InvalidReturnType@5:4-5:24: Return type '"not a bool"' is not compatible with declared 'bool'
