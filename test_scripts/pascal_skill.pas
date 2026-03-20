{ ---
{ name: pascal_algorithm_solver
{ description: Pascal 算法求解器
{ tags: [pascal, algorithm, solver]
{ command_template: fpc {filepath} -o {output}
{ args:
{   output:
{     type: string
{     description: 输出文件
{     required: true
{ --- }
program Solver;
begin
  writeln('Pascal Algorithm Solver');
end.
