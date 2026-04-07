===source===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(42); }
===expect===
