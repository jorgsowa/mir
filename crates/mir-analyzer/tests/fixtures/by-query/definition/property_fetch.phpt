===cursor===
definition
===file===
<?php
final class Point {
    public int $x = 0;
}
$p = new Point();
echo $p-><CURSOR>x;
===expect===
test.php@3:4-3:21
