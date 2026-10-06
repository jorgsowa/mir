===description===
!isset short-circuit with || operator — property chain on narrowed variable
Variable used with property access in RHS should be narrowed as defined from !isset() LHS
===config===
<mir>
  <issueHandlers>
    <MixedPropertyFetch errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
if (!isset($obj) || $obj->prop->method()) {
//                  ^^^^^^^^^^^^^^^^^^^^ MixedMethodCall: Method method() called on mixed type
    // After fix: $obj should be narrowed as defined in RHS
}
