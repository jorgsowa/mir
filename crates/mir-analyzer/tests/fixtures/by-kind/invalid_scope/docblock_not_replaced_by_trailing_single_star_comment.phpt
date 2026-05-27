===description===
/** @var docblock */ is not replaced by a subsequent /* comment */ before a statement
===file===
<?php
class View {
    public string $title = '';
}
/** @var View $this */
/* trailing comment */
$this->title = 'About Us';
/** @mir-check $this is View */
echo $this->title;
===expect===
