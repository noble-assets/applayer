package main

import (
	"encoding/json"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/core"
	coretypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/params"
)

// ChainConfigWithEvolve defines a params.ChainConfig modified to support Evolve.
type ChainConfigWithEvolve struct {
	*params.ChainConfig
	Evolve *EvolveChainConfig `json:"evolve,omitempty"`
}

// GenesisWithEvolve defines a core.Genesis modified to support Evolve.
type GenesisWithEvolve struct {
	*core.Genesis
	Config *ChainConfigWithEvolve `json:"config"`
}

// https://github.com/evstack/ev-reth/blob/main/crates/node/src/config.rs#L5-L11
type EvolveChainConfig struct {
	BaseFeeSink *common.Address `json:"baseFeeSink,omitempty"`
	MintAdmin   *common.Address `json:"mintAdmin,omitempty"`
}

// MarshalJSON is a modified version of the core.Genesis MarshalJSON function
// that supports the Evolve compatible ChainConfig.
func (g GenesisWithEvolve) MarshalJSON() ([]byte, error) {
	type Genesis struct {
		Config        *ChainConfigWithEvolve                         `json:"config"`
		Nonce         math.HexOrDecimal64                            `json:"nonce"`
		Timestamp     math.HexOrDecimal64                            `json:"timestamp"`
		ExtraData     hexutil.Bytes                                  `json:"extraData"`
		GasLimit      math.HexOrDecimal64                            `json:"gasLimit"   gencodec:"required"`
		Difficulty    *math.HexOrDecimal256                          `json:"difficulty" gencodec:"required"`
		Mixhash       common.Hash                                    `json:"mixHash"`
		Coinbase      common.Address                                 `json:"coinbase"`
		Alloc         map[common.UnprefixedAddress]coretypes.Account `json:"alloc"      gencodec:"required"`
		Number        math.HexOrDecimal64                            `json:"number"`
		GasUsed       math.HexOrDecimal64                            `json:"gasUsed"`
		ParentHash    common.Hash                                    `json:"parentHash"`
		BaseFee       *math.HexOrDecimal256                          `json:"baseFeePerGas"`
		ExcessBlobGas *math.HexOrDecimal64                           `json:"excessBlobGas"`
		BlobGasUsed   *math.HexOrDecimal64                           `json:"blobGasUsed"`
	}
	var enc Genesis
	enc.Config = g.Config
	enc.Nonce = math.HexOrDecimal64(g.Nonce)
	enc.Timestamp = math.HexOrDecimal64(g.Timestamp)
	enc.ExtraData = g.ExtraData
	enc.GasLimit = math.HexOrDecimal64(g.GasLimit)
	enc.Difficulty = (*math.HexOrDecimal256)(g.Difficulty)
	enc.Mixhash = g.Mixhash
	enc.Coinbase = g.Coinbase
	if g.Alloc != nil {
		enc.Alloc = make(map[common.UnprefixedAddress]coretypes.Account, len(g.Alloc))
		for k, v := range g.Alloc {
			enc.Alloc[common.UnprefixedAddress(k)] = v
		}
	}
	enc.Number = math.HexOrDecimal64(g.Number)
	enc.GasUsed = math.HexOrDecimal64(g.GasUsed)
	enc.ParentHash = g.ParentHash
	enc.BaseFee = (*math.HexOrDecimal256)(g.BaseFee)
	enc.ExcessBlobGas = (*math.HexOrDecimal64)(g.ExcessBlobGas)
	enc.BlobGasUsed = (*math.HexOrDecimal64)(g.BlobGasUsed)
	return json.Marshal(&enc)
}
