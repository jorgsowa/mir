===description===
RawObjectIteration does NOT fire when yield-from target is typed as Traversable.
===file===
<?php
function gen(\Traversable $t): \Generator {
    yield from $t;
}
===expect===
