===description===
MixedArrayOffset fires when a mixed-typed class property is used as the array key
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Router {
    /** @var mixed */
    public $route = 'home';

    public function dispatch(): void {
        $handlers = ['home' => 'HomeHandler', 'about' => 'AboutHandler'];
        echo $handlers[$this->route];
//                     ^^^^^^^^^^^^ MixedArrayOffset: Mixed type used as array offset
    }
}
