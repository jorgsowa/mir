===description===
InvalidCatch does NOT fire for \Error or its subclasses, which implement Throwable via the Error hierarchy.
===config===
suppress=UnusedVariable,MissingThrowsDocblock
===file===
<?php
try {
    throw new \Error("fail");
} catch (\Error $e) {
    echo $e->getMessage();
} catch (\TypeError $e) {
    echo $e->getMessage();
}
===expect===
UnreachableCatch@6:9-6:19: Catch block for 'TypeError' is unreachable — already caught by 'Error'
