===description===
Only implements one requirement
===file===
<?php
use ImplementationRequirementsTraitImposesImplementationRequirements;
use ImplementationRequirementsBaseA;

class Invalid implements A {
//                       ^ UndefinedClass: Class A does not exist
    use ImposesImplementationRequirements;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedTrait: Trait ImposesImplementationRequirements does not exist
}
