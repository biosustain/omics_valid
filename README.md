# omics_valid

This repository serves two purposes:

1. Define specifications for our OMICS formats
2. Provide a validator to ease the use of the specifications.

## Installation

Binaries for Linux, Max and Windows will be generated in the future. For now, it has to be installed from source with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

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
Line 4: E0X97 invalid Uniprot ID
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
omics_valid --format met --model tests/iCLAU786.xml tests/uni_tidy.csv
```

would output:

```
Line 6: clearly_not_a_metabolite not in model!
```

