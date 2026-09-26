===description===
A same-named function in another namespace is not a reference.
===cursor===
references
===file:src/a.php===
<?php
namespace A;

function helper(): int { return 1; }
echo helper();
===file:src/b.php===
<?php
namespace B;

function helper(): int { return 2; }
echo hel<CURSOR>per();
===expect===
src/b.php@5:5-5:11
