===source===
<?php
function test(): void {
    echo $_GET['x'];
}
===expect===
TaintedHtml: echo $_GET['x'];
