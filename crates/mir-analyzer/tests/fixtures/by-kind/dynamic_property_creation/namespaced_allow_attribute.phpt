===description===
In a namespace, an unqualified `#[AllowDynamicProperties]` names that namespace's class and does not allow dynamic properties.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
namespace App;

#[AllowDynamicProperties]
//^^^^^^^^^^^^^^^^^^^^^^ UndefinedAttributeClass: Attribute class App\AllowDynamicProperties does not exist
class Loose {}

#[\AllowDynamicProperties]
class Allowed {}

function writes(Loose $a, Allowed $b): void {
    $a->x = 1;
//  ^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property App\Loose::$x is deprecated
    $b->x = 1;
}
