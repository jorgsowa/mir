===description===
handles single-star @var comment for $this in global scope
===file===
<?php
class View {
    public string $title = '';
}
/* @var View $this */
$this->title = 'About Us';
/** @mir-check $this is View */
echo $this->title;
===expect===
