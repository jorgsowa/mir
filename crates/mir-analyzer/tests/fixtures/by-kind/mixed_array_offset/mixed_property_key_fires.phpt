===description===
MixedArrayOffset fires when a mixed-typed class property is used as the array key
===config===
suppress=MissingPropertyType
===file===
<?php
class Router {
    /** @var mixed */
    public $route = 'home';

    public function dispatch(): void {
        $handlers = ['home' => 'HomeHandler', 'about' => 'AboutHandler'];
        echo $handlers[$this->route];
    }
}
===expect===
MixedArrayOffset@8:23-8:35: Mixed type used as array offset
