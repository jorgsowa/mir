===description===
`return <expr>;` inside a generator sets `Generator::getReturn()`'s value, not
the generator object — a bare `: Generator` native hint must not be checked
against the returned value.
===file===
<?php
function gen(): \Generator {
    yield 'a';
    return true;
}
===expect===
