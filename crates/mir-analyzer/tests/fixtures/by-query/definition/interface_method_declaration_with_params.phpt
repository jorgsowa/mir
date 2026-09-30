===description===
An interface method declaration with parameters resolves by its name, not a parameter.
===cursor===
definition
===file===
<?php
interface Repo {
    public function &fi<CURSOR>nd(int $id, string $scope = 'all'): ?object;
}
===expect===
test.php@3:4-3:67
