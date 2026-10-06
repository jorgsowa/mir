===description===
A @var property naming a missing class does not cascade into InvalidArgument or InvalidPropertyAssignment.
===file===
<?php
namespace App;

class Holder {
    /** @var Nope */
    public $prop;
//  ^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'App\Nope' does not exist
    public int $count = 0;

    public function go(): void {
        echo strlen($this->prop);
        $this->count = $this->prop;
    }
}
