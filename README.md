# omics_valid

This repository serves two purposes:

1. Define specifications for our OMICS formats.
2. Provide a validator to ease the use of the specifications.

#### Table of contents
<!--ts-->
   * [Installation](#installation)
      * [Building from source](#building-from-source)
   * [Specifications](#supported-specifications)
      * [Proteomics](#proteomics)
      * [Tidy Proteomics](#tidy-proteomics)
      * [Metabolomics](#metabolomics)
   * [Usage](#usage)
<!--te-->

## Installation

Binaries for Linux, Mac and Windows are released on every tag.

1. Go to the [releases page](https://github.com/biosustain/omics_valid/releases/).
2. Look for your platform in the file names (check for _apple_ or _windows_ under _Assets_) and download the file:
	- If Linux, you probably want the file which has `gnu` in the name.
	- If Mac, there is only one file.
	- If Windows, you probably want the `.zip` file.
3. Unpack it, a binary file `omics_valid` should have been extracted.
4. (Optional) Put the extracted file `omics_valid` under your PATH.

### Building from source

Install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and run

```
git clone https://github.com/biosustain/omics_valid.git
cd omics_valid
cargo install --path .
```

## Supported specifications

### Proteomics
Protein CSV **without header** in the form

```csv
UNIPROT_ID,NUMBER_VALUE_SAMPLE1,NUMBER_VALUE_SAMPLE2
```

with an arbitrary number of samples. It will report:
* Invalid Uniprot IDs.

Example:

```csv
Q00496,100001,21283
Q7B2Q4,123.3444,0
E0X9C7,10.2,21283
E0X97,1001,21283
E0X9C7,1000.2,23131
```

Running the command

```shell
omics_valid --format prot tests/uni.csv
```

would output

```
1 lines[4]: E0X97 invalid Uniprot ID
```

since "E0X97" is not a valid Uniprot ID.

### Tidy Proteomics

Protein CSV  in the following tidy (see tidy data, [Hadley Wickham, 2014](https://www.jstatsoft.org/article/view/v059i10)) form:

```csv
uniprot,sample,value
UNIPROT_ID,SAMPLE_NAME,NUMBER_VALUE
```

It will report:
* Invalid Uniprot IDs.
* Empty samples names.

Example:

```csv
uniprot,sample,value
Q00496,cauto_h2,100001
Q7B2Q4,cauto_h2,100.2
E0X9C7,SIM3,203
```

Running the command

```shell
omics_valid --format tidy_prot tests/uni_tidy.csv
```

won't output anything since the file is properly following the specification.

### Metabolomics
Metabolomics CSV  in the following tidy (see tidy data, [Hadley Wickham, 2014](https://www.jstatsoft.org/article/view/v059i10)) form:

```csv
met_id,sample,value
METABOLITE_IDENTIFIER,SAMPLE_NAME,NUMBER_VALUE
```

It will report:
* Identifier not found in the supplied SBML model.
* Empty samples names.

Example:

```csv
met_id,sample,value
glc__D,SIM1,2
cpd00067,SIM3,1032
clearly_not_a_metabolite,SIM1,2921
acon_C,SIM1,18
MNXM83,SIM2,317
```

Running the command

```shell
omics_valid --format met --model tests/iCLAU786.xml tests/met_tidy.csv
```

would output:

```
1 lines[4]: clearly_not_a_metabolite not in model!
```

### Transcriptomics

RNA files for iModulon. These are experiments from SRA or local files.

```csv
Experiment,LibraryLayout,Platform,Run,R1,R2
String,Single|Paired,ILLUMINA|PACBIO_SMRT|ETC,None|Number,None|path/to/file,None|path/to/file
```

It may contain other fields. The validator will check the following (taken from [modulome-workflow](https://github.com/avsastry/modulome-workflow/tree/65c5bd3c9facef6a41899429403c531923aa5204/2_process_data#setup)):

1. `Experiment`: For public data, this is your SRX ID. For local data, data should be named with a standardized ID (e.g. ecoli_0001)
1. `LibraryLayout`: Either PAIRED or SINGLE
1. `Platform`: Usually ILLUMINA, ABI_SOLID, BGISEQ, or PACBIO_SMRT
1. `Run`: One or more SRR numbers referring to individual lanes from a sequencer. This field is empty for local data.
1. `R1`: For local data, the complete path to the R1 file. If files are stored on AWS S3, filenames should look like `s3://<bucket/path/to>.fastq.gz`. `R1` and `R2` columns are empty for public SRA data.
1. `R2`: Same as R1. This will be empty for SINGLE end sequences.

Additionally, the FASTQ files in R1 and R2 will be checked if present for possible format errors.

```shell
omics_valid -f rna tests/rna.csv
```

would output

```
1 lines[35]: ./tests/data/some.fastq: Declared FASTQ path does not exist!
1 lines[36]: ./tests/data/some.fastq: Declared FASTQ path does not exist!;	Inconsistent experiment: R1 and R2 did not match the LibraryLayout! (assuming local data since field 'Run' is empty)
1 lines[38]: ./tests/invalid.fastq: failure reading FASTQ! One record is incorrect
```

As can be seen, when more than one error is found in a single record,
the errors are concatenated with a ";\t".

### Usage

```shell
$ omics_valid --help
Usage: omics_valid [<file>] [-f <format>] [-m <model>] [-v]

Omics format validator.

Positional Arguments:
  file              input omics file.

Options:
  -f, --format      format of the file. Currently supported: {prot, tidy_prot,
                    met, rna}
  -m, --model       path to SBML model file, used for metabolite verification
  -v, --version     display the version
  --help            display usage information
```
