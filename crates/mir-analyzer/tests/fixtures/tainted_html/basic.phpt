===source===
<?php
function test(): void {
    echo $_GET['x'];
}
===expect===
TaintedHtml: <no snippet>
