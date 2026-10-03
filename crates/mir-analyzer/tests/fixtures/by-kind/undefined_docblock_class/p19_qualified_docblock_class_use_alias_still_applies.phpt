===description===
FP-P19 control: a qualified docblock class name whose first segment IS
`use`-imported must still resolve through that alias — guards against the
`resolve_type_name` fix accidentally falling through to namespace-relative
resolution for names the alias branch should have already handled.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Item.php===
<?php
namespace Vendor\Lib;

class Item {}
===file:Container.php===
<?php
namespace App;

use Vendor\Lib as Lib;

class Container {
    /** @var Lib\Item */
    private $item;
}
===expect===
