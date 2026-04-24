===file:Printable.php===
<?php
interface Printable {
    public function print(): void;
}
===file:Doc.php===
<?php
class Doc implements Printable {
    public function print(): void { echo "doc"; }
}
===file:Printer.php===
<?php
function render(Printable $p): void { $p->print(); }
function test(): void {
    render(new Doc());
}
===expect===
