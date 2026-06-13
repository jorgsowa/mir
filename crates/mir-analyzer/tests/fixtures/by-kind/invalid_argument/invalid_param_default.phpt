===description===
Invalid param default
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
function f(int $p = false) {}
===expect===
