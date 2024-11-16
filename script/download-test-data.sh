#!/bin/bash

function download() {
	curl -L "${1}" -o "${2}" && \
		sha1sum 
}


function main() {

	hash curl &>/dev/null || {
		echo "curl does not exist with \$PATH." >&2;
		return 1;
	}
	hash gunzip &>/dev/null || {
		echo "gunzip does not exist with \$PATH." >&2;
		return 1;
	};
	hash sha1sum &>/dev/null || {
		echo "sha1sum does not exist with \$PATH." >&2;
		return 1;
	};

	local REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
	local OUTPUT_DIR="${REPO_ROOT}/test_fixtures/test_data_unzipped"
	
	if [[ ! -d ${OUTPUT_DIR} ]]; then
		mkdir -p "${OUTPUT_DIR}" || {
			echo "failed to create directory: ${OUTPUT_DIR}" >&2;
			return 1;
		}
	fi

	local -i rc=0;
	pushd "${OUTPUT_DIR}"
		local shasum="728351dab49d8eedf5f56e37f04719cb8b697064 *GeodTest.dat.gz
e93118f24967c6c0905ace447c43b958a2d1353e *GeodTest-short.dat.gz
d510574d4e2cd6fe29198c3da6863ee4a430a59f *GeodTest.dat
b9afd21c0a4da787f9434d9028cf00784c3ca1eb *GeodTest-short.dat"
		if [[ ! -r checks.sha1sum ]]; then
			echo "${shasum}" > checks.sha1sum
		fi
		
		if [[ ! -r 'GeodTest.dat.gz' ]]; then
			curl -L 'https://sourceforge.net/projects/geographiclib/files/testdata/GeodTest.dat.gz' -o 'GeodTest.dat.gz' || {
				rc=1
				echo "failed to download GeodTest.dat.gz" >&2;
			};
		fi
		if [[ ! -r 'GeodTest-short.dat.gz' ]]; then
			curl -L 'https://sourceforge.net/projects/geographiclib/files/testdata/GeodTest-short.dat.gz' -o 'GeodTest-short.dat.gz' || {
				rc=1
				echo "failed to download GeodTest-short.dat.gz" >&2;
			};
		fi
		if [[ ! -r 'GeodTest.dat' ]]; then
			gunzip -c < 'GeodTest.dat.gz' > 'GeodTest.dat' || {
				rc=1
				echo "failed to unzip GeodTest.dat"
			};
		fi
		if [[ ! -r 'GeodTest-short.dat' ]]; then
			gunzip -c < 'GeodTest-short.dat.gz' > 'GeodTest-short.dat' || {
				rc=1
				echo "failed to unzip GeodTest-short.dat"
			};
		fi
		sha1sum -c 'checks.sha1sum' || {
			rc=1
			echo "failed to verify downloads, sha1sums failed" >&2;
		};
	popd
	return $rc;
}

main
