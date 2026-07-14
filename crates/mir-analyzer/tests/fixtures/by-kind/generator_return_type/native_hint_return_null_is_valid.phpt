===description===
`return null;` inside a real generator (contains `yield`) is valid against a
bare `: Generator` hint — `TReturn` is unconstrained without generics, unlike
a non-generator function merely declared to return `Generator`.
===file===
<?php
function gen(): Generator {
    yield from [2];
    return null;
}
===expect===
