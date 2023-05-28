// #[derive(Debug)]
// pub struct Program {
// 	filenum: usize,
// 	truenum: usize,
// }

// print_result = $*.first != '-n' or $*.shift
// expr = $*.shift or abort "usage: #{base($0)} [-n] expr [paths...]"
// dirs = $*.dup.then { _1.empty? ? ['.'] : _1 }
// $*.clear
// $contents = Hash.new { |h,k| h[k] =
//   File.read(k).encode(encoding: 'utf-8', invalid: :replace)
// }
// $filenum = -1
// $truenum = 0
// dirs.each do |dir|
//   $start = dir
//   Find.find dir do |path|
//     $path = path
//     $filenum += 1
//     a = eval expr
//     puts path if a and print_result
//     $truenum += 1 if a
//   end
// end
