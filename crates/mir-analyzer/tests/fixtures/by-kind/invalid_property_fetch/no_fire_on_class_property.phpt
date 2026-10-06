===description===
InvalidPropertyFetch does NOT fire when accessing an existing property on a concrete class instance.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Point {
    public int $x;
    public int $y;
    public function __construct(int $x, int $y) {
        $this->x = $x;
        $this->y = $y;
    }
}

$p = new Point(1, 2);
$px = $p->x;
$py = $p->y;
