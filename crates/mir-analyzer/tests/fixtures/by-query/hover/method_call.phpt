===cursor===
hover
===file===
<?php
final class Repo {
    /** Finds all ids. */
    public function all(): array { return []; }
}
$ids = (new Repo())->a<CURSOR>ll();
===expect===
type: array
docstring: Finds all ids.
definition: test.php@4:4-4:47
