===description===
A fully qualified \define('NAME', value) registers NAME like define().
===file===
<?php
\define('FQ_DEFINED', 1);
function readFqDefined(): int {
    return FQ_DEFINED;
}
