===cursor===
symbol
===file===
<?php
final class Repo {
    public function all(): array { return []; }
}
$ids = (new Repo())->a<CURSOR>ll();
===expect===
kind: method call Repo::all
type: array
