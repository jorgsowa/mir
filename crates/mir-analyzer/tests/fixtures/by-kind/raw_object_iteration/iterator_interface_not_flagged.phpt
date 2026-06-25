===description===
RawObjectIteration does NOT fire when yield-from target implements Iterator.
===file===
<?php
function gen(\Iterator $it): \Generator {
    yield from $it;
}
===expect===
