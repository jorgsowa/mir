===description===
finally sees a value assigned partway through try, before a call that may throw
===file===
<?php
function make(): object { return new stdClass(); }

function f(): void {
    $c = false;
    try {
        $c = null;
        $c = make();
    } finally {
        echo null === $c ? 'none' : 'some';
    }
}
