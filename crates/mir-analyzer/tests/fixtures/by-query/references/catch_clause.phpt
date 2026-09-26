===description===
A class named in a `catch` clause counts as a reference.
===cursor===
references
===file===
<?php
final class AppError extends RuntimeException {}
try {
    throw new AppEr<CURSOR>ror();
} catch (AppError $e) {
}
===expect===
test.php@4:14-4:22
test.php@5:9-5:17
