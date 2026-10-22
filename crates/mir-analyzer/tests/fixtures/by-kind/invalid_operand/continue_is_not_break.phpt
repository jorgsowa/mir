===description===
Continue is not break
===file===
<?php
                    switch(2) {
                        case 2:
                            echo "two
";
                            continue 2;
//                          ^^^^^^^^ CompileError: Compile error: Cannot 'continue' 2 levels [break-out-of-loop]
                    }
