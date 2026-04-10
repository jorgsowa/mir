===source===
<?php
function test(): void {
    $name = $_POST['name'];
    echo $name;
}
===expect===
TaintedHtml: <no snippet>
