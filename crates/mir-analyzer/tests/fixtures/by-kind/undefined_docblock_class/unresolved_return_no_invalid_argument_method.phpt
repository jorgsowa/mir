===description===
A method @return naming a missing class is reported once and does not cascade into InvalidArgument.
===file===
<?php
namespace App;

class Client {
    /** @return Body of the request result */
    protected function fetch() { return 'x'; }
//                     ^^^^^ UndefinedDocblockClass: Docblock type 'App\Body' does not exist

    public function run(): mixed {
        return json_decode($this->fetch());
    }
}
