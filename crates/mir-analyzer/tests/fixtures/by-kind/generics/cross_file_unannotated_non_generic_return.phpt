===description===
cross-file unannotated return on non-generic class still infers from body
===file:counter.php===
<?php
class Counter {
    public function answer() { return 42; }
}
===file:app.php===
<?php
function app(): void {
    $c = new Counter();
    $result = $c->answer();
    echo $result;
}
===expect===
