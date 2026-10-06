===description===
A docblock class that exists but is incompatible with the parameter is still an InvalidArgument.
===file===
<?php
namespace App;

class Body {}

class Client {
    /** @return Body */
    protected function fetch() { return new Body(); }

    public function run(): mixed {
        return json_decode($this->fetch());
//                         ^^^^^^^^^^^^^^ InvalidArgument: Argument $json of json_decode() expects 'string', got 'App\Body'
    }
}
