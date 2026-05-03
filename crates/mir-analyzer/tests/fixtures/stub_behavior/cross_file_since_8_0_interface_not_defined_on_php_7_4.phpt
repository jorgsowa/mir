===description===
cross file since 8 0 interface not defined on php 7 4
===config===
php_version=7.4
===file:Printable.php===
<?php
class Label implements \Stringable {
    private string $text;
    public function __construct(string $value) {
        $this->text = $value;
    }
    public function __toString(): string { return $this->text; }
}
===file:App.php===
<?php
$label = new Label('hello');
echo $label;
===expect===
Printable.php: UndefinedClass@2:23: Class Stringable does not exist
===ignore===
TODO
