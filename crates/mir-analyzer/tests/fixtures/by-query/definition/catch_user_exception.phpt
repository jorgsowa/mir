===description===
A user exception in a `catch` clause lands on its declaration.
===cursor===
definition
===file===
<?php
final class AppError extends RuntimeException {}
try {
    echo 1;
} catch (AppEr<CURSOR>ror $e) {
}
===expect===
test.php@2:6-2:48
