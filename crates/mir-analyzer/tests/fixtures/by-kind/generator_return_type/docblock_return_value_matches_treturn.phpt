===description===
A `return <expr>;` compatible with the declared `@return Generator<...>`'s
`TReturn` (4th type param) slot is valid.
===file===
<?php
/** @return Generator<int, string, mixed, bool> */
function gen(): \Generator {
    yield 'a';
    return true;
}
===expect===
