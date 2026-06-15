===description===
Save-restore pattern: variable assigned before try and read only in finally is not reported as unused
===file===
<?php
function withConstraints(callable $callback): mixed {
    $previous = getGlobal();
    setGlobal(false);
    try {
        return $callback();
    } finally {
        setGlobal($previous);
    }
}

/** @return bool */
function getGlobal() { return true; }
/** @param bool $v */
function setGlobal($v): void { echo (int)$v; }
===expect===
